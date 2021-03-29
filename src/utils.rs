/*Built-in*/
use std::cmp;
use std::io::Cursor;

/* Third-Party crates */
use bmp::{Image, Pixel};
use printpdf::*;
use barcoders::sym::code128::*;

pub struct PageSize {
    pub width: f64,
    pub height: f64,
    pub margin_width: f64,
    pub margin_height: f64
}

#[allow(dead_code)]
#[allow(non_snake_case)]
impl PageSize {
    pub fn A1() -> PageSize {
        PageSize {
            width: 594.0,
            height: 841.0,
            margin_width: 10.0,
            margin_height: 10.0
        }
    }
    pub fn A2() -> PageSize {
        PageSize {
            width: 420.0,
            height: 594.0,
            margin_width: 10.0,
            margin_height: 10.0
        }
    }
    pub fn A3() -> PageSize {
        PageSize {
            width: 297.0,
            height: 420.0,
            margin_width: 10.0,
            margin_height: 10.0
        }
    }
    pub fn A4() -> PageSize {
        PageSize {
            width: 210.0,
            height: 297.0,
            margin_width: 10.0,
            margin_height: 10.0
        }
    }
    pub fn A5() -> PageSize {
        PageSize {
            width: 148.0,
            height: 210.0,
            margin_width: 10.0,
            margin_height: 10.0
        }
    }
}    

pub struct Table {
    pub rows: Vec<Vec<String>>,
    pub columns: Vec<Column>,
    pub position_y: f64,
    pub max_columns: usize,
    pub borders: bool,
    pub row_height: f64
}

impl Table {
    pub fn default(position: f64) -> Table {
        return Table {
            columns: vec![Column {width: 6}, Column {width: 2}, Column {width: 2}, Column {width: 2}],
            rows: Vec::<Vec<String>>::new(),
            position_y: position,
            max_columns: 12,
            borders: false,
            row_height: 7.5
        }
    }
    pub fn set_borders(&mut self, borders_on: bool) {
        self.borders = borders_on;
    }
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }
    pub fn set_columns(&mut self, columns: Vec<Column>) {
        self.columns = columns;
    }
    pub fn set_columns_len(&mut self, columns: usize) {
        self.max_columns = columns;
    }
    pub fn set_row_height(&mut self, row_height: f64) {
        self.row_height = row_height;
    }
}

pub struct Column {
    pub width: usize
}

impl Column {
    #[allow(dead_code)]
    pub fn default() -> Column {
        Column {
            width: 1
        }
    }
}

pub fn calculate_column_coordinates(page_size: &PageSize, column_index: usize, columns: usize, y: f64) -> (f64, f64) {
    if column_index >= columns {
        panic!("Column Index cannot be larger or equal than the number of columns");
    }

    let inner_width = page_size.width - (page_size.margin_width * 2.0);
    let inner_height = page_size.height - (page_size.margin_height * 2.0);

    let column_size = inner_width / (columns as f64);
    let x = page_size.margin_width + (column_size * column_index as f64);
    let y = inner_height.min(y);

    (x, y)
}

pub fn calculate_border_points(page_size: &PageSize, table: &Table, column_index: usize, row_num: usize) -> Vec<(Point, bool)> {
    if row_num >= table.rows.len() {
        panic!("Row index cannot be larger or equal to the number of rows in the table");
    }
    if column_index >= table.columns.len() {
        panic!("Column Index cannot be larger or equal than the number of columns");
    }

    let border_padding = table.row_height * 0.5;
    let inner_width = page_size.width - (page_size.margin_width * 2.0);
    let column_size = inner_width / (table.max_columns as f64);
    let y: f64 = table.position_y - border_padding - (row_num as f64 * table.row_height);
    let x = page_size.margin_width + table.columns.iter().take(column_index).map(|w| (w.width as f64) * column_size).sum::<f64>();
    let right_x = page_size.margin_width + table.columns.iter().take(column_index + 1).map(|w| (w.width as f64) * column_size).sum::<f64>();

    vec![
        (Point::new(Mm(x), Mm(y)), false),
        (Point::new(Mm(right_x), Mm(y)), false),
        (Point::new(Mm(right_x), Mm(y - table.row_height)), false),
        (Point::new(Mm(x), Mm(y - table.row_height)), false),
    ]
}

pub fn calculate_cell_coordinates(page_size: &PageSize, table: &Table, column_index: usize, row_num: usize) -> (f64, f64) {
    if row_num >= table.rows.len() {
        panic!("Row index cannot be larger or equal to the number of rows in the table");
    }
    if column_index >= table.columns.len() {
        panic!("Column Index cannot be larger or equal than the number of columns");
    }
    
    let border_padding = match table.borders {
        true => table.row_height * 0.25,
        false => 0.0
    };
    let cell_padding = match table.borders {
        true => 1.0,
        false => 0.0
    };

    let inner_width = page_size.width - (page_size.margin_width * 2.0);
    let column_size = inner_width / (table.max_columns as f64);
    let y: f64 = table.position_y - ((row_num + 1) as f64 * table.row_height) - cell_padding;
    let x = page_size.margin_width + table.columns.iter().take(column_index).map(|w| (w.width as f64) * column_size).sum::<f64>() + border_padding;
    (x, y)
}

pub fn add_table(table: &mut Table, page_size: &PageSize, doc: &PdfDocumentReference, current_layer_ref: PdfLayerReference, y: f64, regular: &IndirectFontRef, bold: &IndirectFontRef) -> (f64, PdfLayerReference) {
    let mut current_y = y;
    let mut page_num = 0;
    let mut current_row = 0;
    let mut print_header = true;
    let mut new_layer_ref = current_layer_ref.clone();
    let headers = table.rows.get(0).unwrap();
    
    
    for (r_index, row) in table.rows.iter().enumerate() {
        if current_y <= (page_size.margin_height + 7.5) {
            page_num += 1;
            let (new_page, new_layer) = doc.add_page(Mm(page_size.width), Mm(page_size.height), page_num.to_string());
            new_layer_ref = doc.get_page(new_page).get_layer(new_layer);
            current_row = r_index;
            print_header = true;
            table.position_y = page_size.height - page_size.margin_height;
        }
        if print_header {
            for (c_index, cell) in headers.iter().enumerate() {
                if table.borders {
                    let line1 = Line {
                        points: self::calculate_border_points(&page_size, &table, c_index, r_index - current_row),
                        is_closed: true,
                        has_fill: false,
                        has_stroke: true,
                        is_clipping_path: false,
                    };
                    new_layer_ref.add_shape(line1);
                }
                let (x, y) = self::calculate_cell_coordinates(&page_size, &table, c_index, r_index - current_row);
                new_layer_ref.use_text(cell,  12.0, Mm(x), Mm(y), bold);
                current_y = y;
            }
            print_header = false;
            if r_index == 0 {
                continue;
            }
        }
        for (c_index, cell) in row.iter().enumerate() {
            if table.borders {
                let line1 = Line {
                    points: self::calculate_border_points(&page_size, &table, c_index, r_index + cmp::min(page_num, 1) - current_row),
                    is_closed: true,
                    has_fill: false,
                    has_stroke: true,
                    is_clipping_path: false,
                };
                new_layer_ref.add_shape(line1);
            }
            let (x, y) = self::calculate_cell_coordinates(&page_size, &table, c_index, r_index + cmp::min(page_num, 1) - current_row);
            new_layer_ref.use_text(cell,  12.0, Mm(x), Mm(y), regular);
            current_y = y;
        }
    }
    (current_y, new_layer_ref)
}

pub fn generate_barcode(content: String, height: u32) -> Image {
    let barcode = Code128::new(content).unwrap();
    let buffer = barcoders::generators::image::Image::image_buffer(height);
    let encoded = barcode.encode();
    let buffer = buffer.generate_buffer(&encoded[..]).unwrap();
    let mut img = Image::new(buffer.width(), height);

    for (x, y, &color) in buffer.enumerate_pixels() {
        img.set_pixel(x, y, Pixel::new(color[0], color[1], color[2]));
    }
    return img;
}

pub fn generate_barcode_for_pdf(content: String, height: u32) -> printpdf::Image {
    let img = self::generate_barcode(content, height as u32);
    let mut tr: Vec<u8> = vec![];
    img.to_writer(&mut tr).unwrap();
    let file = Cursor::new(tr);
    match printpdf::Image::try_from(image::bmp::BmpDecoder::new(file).unwrap()) {
        Ok(x) => x,
        Err(_x) => {
            panic!("Can't open image");
        }
    }
}
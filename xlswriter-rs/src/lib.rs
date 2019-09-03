#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::ffi::CString;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn create_workbook(path: &str) -> () {
  let file_path = CString::new(path).expect("CString::new failed");
  unsafe {
    workbook_new(file_path.as_ptr());
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_sanity() {
    println!("Testing the sanity of the project");

    create_workbook("/tmp/test_excel.xlsx");
    //     lxw_worksheet *worksheet = workbook_add_worksheet(workbook, NULL);
    //     int row = 0;
    //     int col = 0;
    //     worksheet_write_string(worksheet, row, col, "Hello me!", NULL);

    // #include "xlsxwriter.h"
    // int main() {
    //     lxw_workbook  *workbook  = workbook_new("myexcel.xlsx");
    //     lxw_worksheet *worksheet = workbook_add_worksheet(workbook, NULL);
    //     int row = 0;
    //     int col = 0;
    //     worksheet_write_string(worksheet, row, col, "Hello me!", NULL);
    //     return workbook_close(workbook);
    // }
  }
}

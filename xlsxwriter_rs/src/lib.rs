#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::ffi::CString;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// *const ::std::os::raw::c_char

pub fn create_workbook(path: &str) -> *mut lxw_workbook {
  println!("Trying to create workbook at: {}", path);
  // TODO: Does exist test - raise error if already exists
  let file_path = CString::new(path).expect("CString::new failed");
  unsafe {
    let result = workbook_new(file_path.as_ptr());
    let book = *result;
    println!("Create WB: {:#?}", *book.sheets);
    return result;
  };
}

pub fn create_sheet(wb: *mut lxw_workbook, name: &str) -> *mut lxw_worksheet {
  let sheet_name = CString::new(name)
    .expect("CString::new failed for create sheet")
    .as_ptr();
  unsafe {
    let result = workbook_add_worksheet(wb, sheet_name);
    let book = *wb;
    println!("Sheets: {:#?}", *book.sheets);
    return result;
  };
}

pub fn write_cell(
  ws: *mut lxw_worksheet,
  row: u32,
  column: u16,
  value: &str,
  format: Option<*mut lxw_format>,
) -> lxw_error {
  let cell = CString::new(value)
    .expect("CString::new failed for create sheet")
    .as_ptr();
  let fmt = unsafe {
    match format {
      Some(fmt) => fmt,
      None => lxw_format_new(),
    }
  };

  unsafe {
    let result = worksheet_write_string(ws, row, column, cell, fmt);
    return result;
  };
}

pub fn write_workbook(wb: *mut lxw_workbook) -> lxw_error {
  unsafe {
    let result = workbook_close(wb);
    result
  }
}

#[test]
fn test_xlsxwriter_unsafe_sanity() {
  println!("Testing the sanity of the project");

  let file_path = CString::new("/tmp/UnsafeTest.xlsx").expect("CString::new failed");

  println!(
    "File Path: {:#?}",
    file_path
      .clone()
      .into_string()
      .expect("Couldn't convert file_path")
  );
  let sheet_name = CString::new("TestSheet").expect("CString::new failed for create sheet");
  let row: lxw_row_t = 1;
  let column: lxw_col_t = 1;
  let cell = CString::new("TestCell").expect("CString::new failed for create cell");

  let fmt = unsafe { lxw_format_new() };
  unsafe { format_set_bold(fmt) };
  unsafe { println!("Get XF Index: {}", lxw_format_get_xf_index(fmt)) };

  let wb = unsafe { workbook_new(file_path.as_ptr()) };
  unsafe {
    let book = *wb;
    println!("Made Workbook: {:#?}", *book.filename);
  }

  let ws = unsafe { workbook_add_worksheet(wb, sheet_name.as_ptr()) };
  println!("Made Worksheet");

  println!("Writing the String: {}", unsafe {
    worksheet_write_string(ws, row, column, cell.as_ptr(), fmt)
  });
  println!("Done Writing. About to close");
  unsafe { println!("Closing the Workbook: {}", workbook_close(wb)) }
}


extern crate subpar;
extern crate calamine;
extern crate chrono;

use calamine::DataType;
use chrono::NaiveDateTime;
use subpar::{SubparError, ExcelObject, MetaWorkbook, FromExcel};

/// A record of sending a durable message.
/// Note - this is not the message itself, just a pointer to where the message lives
/// and where it was sent to. This is for situations where the user updates their email address,
/// we want a record of where it was actually sent, not the new address.
// pub struct SentMessage {
//   /// The global ID
//   guid: String,
//   /// The system user that the message was sent to
//   user_id: String,
//   /// The meduim the message was sent. Currently we will have email, sms and snail.
//   contact_method: String,
//   /// Where the message was sent to.
//   /// Currently debating if this should be an ID (postal address), or the actual string(email, phone)
//   contact_address: String,
//   /// Identify the genre of message. Invoice, Pick-Up.
//   message_type: String,
//   /// The id of the item the user was notified about
//   message_body_id: String,
//   /// When did it hit the mailbox
//   date_sent: NaiveDateTime,
// }

// pub struct Submission {
//   accession_number: String,
//   submitting_org: String,
//   service_type: String,
//   service_line_items: Option<String>,
//   species: String,
//   pet_name: Option<String>,
//   slides: Option<i32>,
//   diagnosis: Option<String>,
//   expenses: Option<String>,
//   price: Option<f64>,
//   invoice_number: Option<i32>,
//   received: NaiveDateTime,
//   finalized: Option<NaiveDateTime>,
//   billed_on: Option<NaiveDateTime>,
//   paid_on: Option<NaiveDateTime>,
// }


// pub struct FullInvoice {
//   guid: String,
//   organization: String,
//   status: String,
//   balance: f64,
//   credits: f64,
//   debits: f64,
//   date_created: NaiveDateTime,
//   due_on: Option<NaiveDateTime>,
//   terms: String,
//   last_modified: NaiveDateTime,
//   sent_to: Vec<SentMessage>,
//   submissions: Vec<Submission>,
//   payments: Vec<Payment>,
// }


#[derive(Debug, Clone, FromExcel)]
pub struct Payment {
  guid: String,
  payer: String,
  payee: String,
  method: String,
  amount: f64,
  comment: Option<String>,
  date_received: NaiveDateTime,
}

#[derive(Debug, Clone, FromExcel)]
pub struct DB {
  // pub sent_messages: Vec<SentMessage>,
  // pub submissions: Vec<Submission>,

  #[subpar(rename="payment", parser="cell_csv_to_vec")]
  pub payment: Vec<Payment>,
}


pub fn cell_csv_to_vec (_cell: DataType) -> Result<Vec<String>, SubparError> {
  Err(SubparError::NotImplemented("cell_csv_to_vec is not yet implemented".to_string()))
}


#[test]
fn test_ctx() {
  let wb = MetaWorkbook::new("../subpar_test/data/test_db.xlsx".to_string());
  let db = DB::from_excel(&ExcelObject::Workbook(wb));
  println!("db:\n{:#?}", db);
}

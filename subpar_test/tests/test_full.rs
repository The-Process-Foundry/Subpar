#[macro_use]
extern crate subpar;
extern crate chrono;

use chrono::NaiveDateTime;
use subpar::FromExcel;

/// A record of sending a durable message.
/// Note - this is not the message itself, just a pointer to where the message lives
/// and where it was sent to. This is for situations where the user updates their email address,
/// we want a record of where it was actually sent, not the new address.
pub struct SentMessage {
  /// The global ID
  guid: String,
  /// The system user that the message was sent to
  user_id: String,
  /// The meduim the message was sent. Currently we will have email, sms and snail.
  contact_method: String,
  /// Where the message was sent to.
  /// Currently debating if this should be an ID (postal address), or the actual string(email, phone)
  contact_address: String,
  /// Identify the genre of message. Invoice, Pick-Up.
  message_type: String,
  /// The id of the item the user was notified about
  message_body_id: String,
  /// When did it hit the mailbox
  date_sent: NaiveDateTime,
}

pub struct Submission {
  accession_number: String,
  submitting_org: String,
  service_type: String,
  service_line_items: Option<String>,
  species: String,
  pet_name: Option<String>,
  slides: Option<i32>,
  diagnosis: Option<String>,
  expenses: Option<String>,
  price: Option<f64>,
  invoice_number: Option<i32>,
  received: NaiveDateTime,
  finalized: Option<NaiveDateTime>,
  billed_on: Option<NaiveDateTime>,
  paid_on: Option<NaiveDateTime>,
}

#[derive(FromExcel)]
pub struct Payment {
  guid: String,
  payer: String,
  payee: String,
  method: String,
  amount: f64,
  comment: Option<String>,
  date_received: NaiveDateTime,
}

pub struct FullInvoice {
  guid: String,
  organization: String,
  status: String,
  balance: f64,
  credits: f64,
  debits: f64,
  date_created: NaiveDateTime,
  due_on: Option<NaiveDateTime>,
  terms: String,
  last_modified: NaiveDateTime,
  sent_to: Vec<SentMessage>,
  submissions: Vec<Submission>,
  payments: Vec<Payment>,
}

pub struct Ctx {
  pub sent_messages: Vec<SentMessage>,
  pub submissions: Vec<Submission>,
  pub payment: Vec<Payment>,
}

// impl Ctx {
//     pub fn empty() -> Ctx {
//         Ctx {
//             sent_messages: Vec::new(),
//             submissions: Vec::new(),
//             payment: Vec::new(),
//         }
//     }
// }

// impl juniper::Context for Ctx {}

// #[juniper::object(
//     description = "The full invoice, including the joined lists of submissions, payments and messages",
//     Context = Ctx
// )]
// impl FullInvoice {
//     fn guid(&self) -> &str {
//         &self.guid.as_str()
//     }
//     fn organization(&self) -> &str {
//         &self.organization.as_str()
//     }
//     fn status(&self) -> &str {
//         &self.status.as_str()
//     }
//     fn balance(&self) -> &f64 {
//         &self.balance
//     }
//     fn credits(&self) -> &f64 {
//         &self.credits
//     }
//     fn debits(&self) -> &f64 {
//         &self.debits
//     }
//     fn date_created(&self) -> &NaiveDateTime {
//         &self.date_created
//     }
//     fn due_on(&self) -> &Option<NaiveDateTime> {
//         &self.due_on
//     }
//     fn terms(&self) -> &str {
//         &self.terms.as_str()
//     }
//     fn last_modified(&self) -> &NaiveDateTime {
//         &self.last_modified
//     }
//     fn sent_to(&self) -> &Vec<SentMessage> {
//         &self.sent_to
//     }
//     fn submissions(&self) -> &Vec<Submission> {
//         &self.submissions
//     }
//     fn payments(&self) -> &Vec<Payment> {
//         &self.payments
//     }
// }

// struct Query;
// graphql_object!(Query: Ctx |&self| {
//   field apiVersion() -> &str {
//     "1.0"
//   }

// //   field full_invoices(&executor) -> FieldResult<Vec<FullInvoice>
// });

// type Schema = juniper::RootNode<'static, Query, juniper::EmptyMutation<Ctx>>;

// fn run_query(ctx: Ctx, query: &str) -> () {
//     println!("Running the query:\n{}\n", query);
//     let (res, error) = juniper::execute(
//         query,
//         None,
//         &Schema::new(Query, juniper::EmptyMutation::new()),
//         &juniper::Variables::new(),
//         &ctx,
//     )
//     .unwrap();
//     println!(
//         "And received the Result:\nOk - {:#?}\nErr - {:#?}\n\n",
//         res, error
//     );
// }

// fn main() {
//     println!("\n\nRunning the test\n");
//     // An query them all
//     run_query(
//         Ctx::empty(),
//         r#"
//       query {
//         full_invoices {
//           guid
//         }
//       }
//     "#,
//     );
// }

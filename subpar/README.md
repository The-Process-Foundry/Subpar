# Subpar

This is an abstraction to be able to work with excel/google sheets data without all the boilerplate of
reading/parsing a spreadsheet. The primary focus is to be able to use Google/Sheets as a UI.  Subpar_derive
uses a Serde-like api to convert to rust objects.

I'm writing this for specific business requirement, which started with keeping a workbook in
Excel and then needing to switch to Google Sheets.

**This not optimized in any way. I'm currently focusing on getting it to work and may refactor the API
 significantly in the future**

## Usage

I'll add this once I stablize the upsert items for Google Sheets

## TODO

This is the list of steps I'm currently working on:

1. Update Documentation
2. Store data cache in workbook? - converting it to be on demand
3. Updating LastModified


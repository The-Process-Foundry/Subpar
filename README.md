# Subpar
A set of macros for serializing/deserializing tabular data.

**THIS IS A WORK IN PROGRESS, DO NOT USE**

Currently, this is being rewritten to abstract the idea of the various types of tables that
can be read. It is meant to eventually be able to read/write, if I have time and/or use for
that functionality.

The old master branch currently holds running code can read from either Excel or Google Sheets.

## Notes

- Google Sheets requires adding a json file somewhere with user credentials
  that can access the requested sheet
- CSV should treat a directory as a spreadsheet

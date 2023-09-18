# Description of the `tsheet` filetype

## General Layout

Rows surrounded by [].

An example row may look like: `[1, 2]` which would contain 1 in column A, and 2 in column B.

The first row is a description of how wide each column should be.
For example:

`[10, 15]`, column A is 10 chars wide, and B is 15 chars wide

an example file my look like

```tsheet
[10, 15]
[1, "a string"]
[(3 + 3), ($a1)]
```

With a proper tsheet reader this would appear something like

| A             | B        |
| ------------- | -------- |
| 1             | a string |
| 6             | 1        |


## Types

There are currently 3 types

- String, text surrounded by "". To put a " in a string precede it with a \\.
- Number,
- Equation, text surrounded by ().
    - To reference another cell use $&lt;COLUMN&gt;&lt;row&gt;, as in the example

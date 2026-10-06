===description===
Column offsets in diagnostics are 0-based (LSP utf-32 convention).
The call site starts at byte offset 8 on line 2 (0-indexed col 8).
===file===
<?php
        \noSuchFn();
//      ^^^^^^^^^^^ UndefinedFunction: Function noSuchFn() is not defined

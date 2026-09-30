===description===
reports return null from void
===file===
<?php
function f(): void {
    return null;
//  ^^^^^^^^^^^^ ParseError: Parse error: A void function must not return a value
}
===expect===

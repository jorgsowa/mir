===description===
reports return null from void
===file===
<?php
function f(): void {
    return null;
}
===expect===
ParseError@3:5-3:17: Parse error: A void function must not return a value

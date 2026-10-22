===description===
Returning a value from a `: void` function, closure or arrow function is an error.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(): void {
    return null;
//  ^^^^^^^^^^^^ InvalidReturnType: Return type 'null' is not compatible with declared 'void'
}
$c = function (): void {
    return 1;
//  ^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'void'
};
$a = fn(): void => 1;
//                 ^ InvalidReturnType: Return type '1' is not compatible with declared 'void'

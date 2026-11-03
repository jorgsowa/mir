===description===
A function whose only call is inside its own body is unused
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
function countdown(int $n): int {
//<^ +2:1 UnusedFunction: Function countdown() is never called
    return $n > 0 ? countdown($n - 1) : 0;
}

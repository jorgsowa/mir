===description===
Inexistent callablein callable string
===file===
<?php
/**
 * @param callable-string $c
 */
function c(string $c): void {
    $c();
}

c("hii");
===expect===
UndefinedFunction@9:3-9:8: Function hii() is not defined

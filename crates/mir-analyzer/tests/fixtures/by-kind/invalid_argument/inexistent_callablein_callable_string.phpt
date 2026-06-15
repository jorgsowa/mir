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
UndefinedFunction@9:2-9:7: Function hii() is not defined

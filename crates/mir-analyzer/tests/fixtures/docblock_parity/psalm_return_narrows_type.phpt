===file===
<?php
class Product {}

/**
 * @return mixed
 * @psalm-return Product
 */
function getProduct(): mixed {
    return new Product();
}

function test(): void {
    getProduct()->missing();
}
===expect===
UndefinedMethod: Method Product::missing() does not exist

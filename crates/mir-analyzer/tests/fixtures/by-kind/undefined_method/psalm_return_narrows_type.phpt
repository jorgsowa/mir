===description===
psalm return narrows type
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
UndefinedMethod@13:4-13:27: Method Product::missing() does not exist

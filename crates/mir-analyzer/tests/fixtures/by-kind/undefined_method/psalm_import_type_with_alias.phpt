===description===
psalm import type with alias
===file===
<?php
class Product {}

/**
 * @psalm-type ProductModel = Product
 */
class ProductRepository {}

/**
 * @psalm-import-type ProductModel as PM from ProductRepository
 * @method PM get()
 */
class ProductService {}

function test(ProductService $s): void {
    $s->get()->missing();
}
===expect===
UndefinedMethod@16:5-16:25: Method Product::missing() does not exist

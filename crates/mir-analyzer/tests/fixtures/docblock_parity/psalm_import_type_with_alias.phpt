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
UndefinedMethod: Method Product::missing() does not exist

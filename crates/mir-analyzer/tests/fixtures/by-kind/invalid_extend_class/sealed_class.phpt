===description===
`@phpstan-sealed A|B` and `@psalm-inheritors` on a class or interface list the only classes that may extend or implement it. Any other subtype is `InvalidExtendClass`.
===file===
<?php
/**
 * @phpstan-sealed AllowedOne|AllowedTwo
 */
abstract class Base
{
}

final class AllowedOne extends Base
{
}

final class AllowedTwo extends Base
{
}

final class Forbidden extends Base
//    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class Forbidden cannot extend sealed Base, which allows only AllowedOne|AllowedTwo
{
}

/**
 * @psalm-inheritors Circle
 */
interface Shape
{
}

final class Circle implements Shape
{
}

final class Square implements Shape
//    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class Square cannot extend sealed Shape, which allows only Circle
{
}

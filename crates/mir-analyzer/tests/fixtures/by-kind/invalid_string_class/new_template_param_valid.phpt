===description===
new with a template-typed variable is not InvalidStringClass — the template's
bound may be a class-string
===file===
<?php
/**
 * @template T of object
 * @param class-string<T> $class
 * @return T
 */
function make(string $class) {
    return new $class();
}
===expect===

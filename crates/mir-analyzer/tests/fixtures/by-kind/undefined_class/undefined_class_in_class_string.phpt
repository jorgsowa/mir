===description===
Undefined class in class string
===file===
<?php
/**
 * @param class-string<SomeClass> $className
 */
function instantiateClass($className) {
    return new $className();
}

// Passing a non-existent class reference
// SHOULD emit UndefinedClass because it's documented as class-string
instantiateClass("NonExistentClass");
===expect===
UndefinedClass@11:18-11:36: Class NonExistentClass does not exist

===description===
Undefined class in class string
===config===
suppress=MissingReturnType
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
//               ^^^^^^^^^^^^^^^^^^ UndefinedClass: Class NonExistentClass does not exist
===expect===

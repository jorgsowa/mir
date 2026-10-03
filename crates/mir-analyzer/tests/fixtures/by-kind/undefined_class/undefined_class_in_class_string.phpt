===description===
Undefined class in class string
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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

===description===
Undefined function in docblock
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param callable-string $callback
 */
function executeCallback($callback) {
    return $callback();
}

// Passing a non-existent function reference in docblock context
// SHOULD emit UndefinedFunction because it's documented as callable
executeCallback("nonExistentFunction");
//              ^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function nonExistentFunction() is not defined
===expect===

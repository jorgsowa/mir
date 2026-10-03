===description===
Array type coercion
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B extends A{}

/**
 * @param  B[]  $b
 * @return void
 */
function fooFoo(array $b) {}
fooFoo([new A()]);
===expect===

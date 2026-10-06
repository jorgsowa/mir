===description===
new with mixed variable is not InvalidStringClass — mixed is already imprecise
(a Mixed* concern), matching the static-call path
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(mixed $value) {
    new $value();
}

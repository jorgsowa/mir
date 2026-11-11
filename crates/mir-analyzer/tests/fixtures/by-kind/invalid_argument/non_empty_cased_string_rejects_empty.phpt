===description===
The `non-empty-` case refinements reject the empty string as well as the wrong case.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-lowercase-string $key */
function cacheKey(string $key): void {}

/** @param non-empty-uppercase-string $verb */
function httpVerb(string $verb): void {}

cacheKey('user.42');
cacheKey('');
//       ^^ InvalidArgument: Argument $key of cacheKey() expects 'non-empty-lowercase-string', got '""'
cacheKey('User.42');
//       ^^^^^^^^^ InvalidArgument: Argument $key of cacheKey() expects 'non-empty-lowercase-string', got '"User.42"'
httpVerb('GET');
httpVerb('');
//       ^^ InvalidArgument: Argument $verb of httpVerb() expects 'non-empty-uppercase-string', got '""'
httpVerb('get');
//       ^^^^^ InvalidArgument: Argument $verb of httpVerb() expects 'non-empty-uppercase-string', got '"get"'

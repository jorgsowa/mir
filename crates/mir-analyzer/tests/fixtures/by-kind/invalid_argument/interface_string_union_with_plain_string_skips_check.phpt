===description===
When the parameter also accepts a plain string (e.g. `string|interface-string<T>`),
a literal string that names neither an existing class nor an interface is NOT
flagged — mirrors class-string's IoC-container-key escape hatch.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param string|interface-string $key
 */
function resolveKey(string $key) {
    return $key;
}

resolveKey('database.connection');
===expect===

===description===
A non-falsy-string is accepted wherever a non-empty-string or string is, including after a !== false guard.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-falsy-string $value */
function takesNonFalsy(string $value): void {}
/** @param non-empty-string $value */
function takesNonEmpty(string $value): void {}
function takesString(string $value): void {}

/**
 * @param non-falsy-string $name
 * @param truthy-string $truthy
 * @param non-falsy-string|false $found
 * @param class-string $class
 */
function run(string $name, string $truthy, $found, string $class): void {
    takesNonFalsy($name);
    takesNonEmpty($name);
    takesString($name);
    takesNonFalsy($truthy);
    takesNonFalsy($class);
    takesNonFalsy($name . 'x');
    if ($found !== false) {
        takesNonFalsy($found);
        takesNonEmpty($found);
    }
}

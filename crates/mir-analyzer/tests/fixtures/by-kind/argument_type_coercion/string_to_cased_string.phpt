===description===
A string of unknown or other case is a coercion to `lowercase-string`; the result of strtolower() is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param lowercase-string $value */
function takesLower(string $value): void {}

/** @param non-empty-lowercase-string $value */
function takesNonEmptyLower(string $value): void {}

/** @param uppercase-string $value */
function takesUpper(string $value): void {}

/**
 * @param non-empty-string $name
 * @param uppercase-string $shout
 */
function run(string $plain, string $name, string $shout): void {
    takesLower($plain);
//             ^^^^^^ ArgumentTypeCoercion: Argument $value of takesLower() expects 'lowercase-string', got 'string' — coercion may fail at runtime
    takesLower($shout);
//             ^^^^^^ ArgumentTypeCoercion: Argument $value of takesLower() expects 'lowercase-string', got 'uppercase-string' — coercion may fail at runtime
    takesLower(strtolower($plain));
    takesLower(mb_strtolower($plain));
    takesNonEmptyLower(strtolower($plain));
//                     ^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $value of takesNonEmptyLower() expects 'non-empty-lowercase-string', got 'lowercase-string' — coercion may fail at runtime
    takesNonEmptyLower(strtolower($name));
    takesUpper(strtoupper($plain));
    takesUpper($shout);
}

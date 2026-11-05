===description===
`filter_var_array` and `filter_input_array` apply their definition per key, and `FILTER_CALLBACK` with a named escaper clears what that escaper does.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function allInts(): void {
    echo implode(',', filter_var_array($_GET, FILTER_VALIDATE_INT));
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $array of implode() expects 'array|null', possibly different type 'array|false|null' provided
    echo implode(',', filter_input_array(INPUT_GET, ['a' => FILTER_VALIDATE_INT, 'b' => ['filter' => FILTER_VALIDATE_FLOAT]]));
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $array of implode() expects 'array|null', possibly different type 'array|false|null' provided
}

function oneRawKey(): void {
    echo implode(',', filter_input_array(INPUT_GET, ['a' => FILTER_VALIDATE_INT, 'b' => FILTER_DEFAULT]));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $array of implode() expects 'array|null', possibly different type 'array|false|null' provided
    echo implode(',', filter_input_array(INPUT_GET));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $array of implode() expects 'array|null', possibly different type 'array|false|null' provided
    echo implode(',', filter_var_array($_GET));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
//                    ^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $array of implode() expects 'array|null', possibly different type 'array|false|null' provided
}

function callback(): void {
    echo filter_var($_GET['x'], FILTER_CALLBACK, ['options' => 'htmlspecialchars']);
    echo filter_var($_GET['x'], FILTER_CALLBACK, ['options' => 'trim']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

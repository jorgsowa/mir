===description===
`+=` stays sticky taint when it can be an array union (`$opts += $_GET`), on
variable and property targets. With a scalar operand it is arithmetic and the
result is a number.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Opts {
    public array $values = [];
}

function variableTarget(array $opts): void {
    $opts += $_GET;
    echo implode(',', $opts);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function propertyTarget(Opts $o): void {
    $o->values += $_GET;
    echo implode(',', $o->values);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function scalarTarget(): void {
    $id = 0;
    $id += $_GET['id'];
    echo $id;
}

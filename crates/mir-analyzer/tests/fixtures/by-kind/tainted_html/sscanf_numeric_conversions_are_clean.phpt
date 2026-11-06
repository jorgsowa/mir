===description===
`sscanf` outputs and result elements from numeric conversions can't carry markup; `%s` ones can.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <PossiblyInvalidArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = (string) $_GET['x'];
sscanf($x, '%d-%x %s', $a, $b, $c);
echo $a;
echo $b;
echo $c;
//<^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
[$d] = sscanf($x, '%d');
echo $d;

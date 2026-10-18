===description===
XPath expressions passed to `DOMXPath::query()`, `DOMXPath::evaluate()` and
`SimpleXMLElement::xpath()` are XPath injection sinks.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function find(DOMXPath $xpath, SimpleXMLElement $xml): void {
    $xpath->query("//user[name='" . $_GET['name'] . "']");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXpath: Tainted XPath query — possible XPath injection
    $xpath->evaluate('count(//' . $_GET['tag'] . ')');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXpath: Tainted XPath query — possible XPath injection
    $xml->xpath('//item[@id=' . $_GET['id'] . ']');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXpath: Tainted XPath query — possible XPath injection
    $xml->xpath('//item[@id=' . (int) $_GET['id'] . ']');
    $xpath->query('//user');
}

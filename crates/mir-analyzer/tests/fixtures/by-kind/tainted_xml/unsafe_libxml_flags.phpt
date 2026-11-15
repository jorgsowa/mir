===description===
User-controlled XML parsed with `LIBXML_NOENT`, `LIBXML_DTDLOAD` or
`LIBXML_PARSEHUGE` through each parsing entry point.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function parse(DOMDocument $doc, XMLReader $reader): void {
    simplexml_load_string($_POST['xml'], 'SimpleXMLElement', LIBXML_NOENT);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXml: Tainted XML parsed with entity expansion, DTD loading or huge documents enabled — possible XXE or XML denial of service
    $doc->loadXML($_POST['xml'], LIBXML_DTDLOAD | LIBXML_NONET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXml: Tainted XML parsed with entity expansion, DTD loading or huge documents enabled — possible XXE or XML denial of service
    $reader->XML($_POST['xml'], null, LIBXML_PARSEHUGE);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXml: Tainted XML parsed with entity expansion, DTD loading or huge documents enabled — possible XXE or XML denial of service
    new SimpleXMLElement($_POST['xml'], options: LIBXML_NOENT);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXml: Tainted XML parsed with entity expansion, DTD loading or huge documents enabled — possible XXE or XML denial of service
}

/** @taint-sink xml $xml */
function parseWithEntities(string $xml): void {}

function viaAnnotation(): void {
    parseWithEntities($_POST['xml']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXml: Tainted XML parsed with entity expansion, DTD loading or huge documents enabled — possible XXE or XML denial of service
}

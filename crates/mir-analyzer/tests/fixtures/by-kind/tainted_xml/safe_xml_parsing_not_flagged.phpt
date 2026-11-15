===description===
Default libxml flags are safe for untrusted XML, and unsafe flags are fine for
XML the program controls.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function parse(DOMDocument $doc, string $trusted): void {
    simplexml_load_string($_POST['xml']);
    simplexml_load_string($_POST['xml'], 'SimpleXMLElement', LIBXML_NONET | LIBXML_NOCDATA);
    $doc->loadXML($_POST['xml']);
    new SimpleXMLElement($_POST['xml']);
    $doc->loadXML($trusted, LIBXML_NOENT);
    simplexml_load_string(file_get_contents(__DIR__ . '/config.xml'), 'SimpleXMLElement', LIBXML_PARSEHUGE);
}

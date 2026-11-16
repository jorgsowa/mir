===description===
libxml flags held in a variable are checked like flags written at the call.
===config===
<mir>
  <issueHandlers>
    <PossiblyFalseArgument errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Importer {
    public function import(): void {
        $xml = (string) file_get_contents('php://input');
        $flags = LIBXML_DTDLOAD | LIBXML_NONET;
        simplexml_load_string($xml, 'SimpleXMLElement', $flags);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXml: Tainted XML parsed with entity expansion, DTD loading or huge documents enabled — possible XXE or XML denial of service
        $doc = new \DOMDocument();
        $entities = LIBXML_NOENT;
        $doc->loadXML($xml, $entities);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedXml: Tainted XML parsed with entity expansion, DTD loading or huge documents enabled — possible XXE or XML denial of service
        $safe = LIBXML_NONET;
        simplexml_load_string($xml, 'SimpleXMLElement', $safe);
    }
}

===description===
DOMDocument::schemaValidateSource() accepts an omitted optional flags argument on PHP 7.4.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php

$document = new DOMDocument();
$schema = '<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" />';

// The libxml flags argument defaults to 0.
$defaultFlags = $document->schemaValidateSource($schema);
$explicitFlags = $document->schemaValidateSource($schema, 0);

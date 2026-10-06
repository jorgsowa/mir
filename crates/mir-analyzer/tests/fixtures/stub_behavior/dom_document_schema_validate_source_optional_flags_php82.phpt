===description===
DOMDocument::schemaValidateSource() retains its required schema source while accepting
the optional flags argument on PHP 8.2.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

$document = new DOMDocument();
$schema = '<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" />';

$defaultFlags = $document->schemaValidateSource($schema);
$namedFlags = $document->schemaValidateSource(source: $schema, flags: 0);
$missingSource = $document->schemaValidateSource();
//               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for schemaValidateSource(): expected 1, got 0

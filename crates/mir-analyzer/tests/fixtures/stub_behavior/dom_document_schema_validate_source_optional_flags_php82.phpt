===description===
DOMDocument::schemaValidateSource() retains its required schema source while accepting
the optional flags argument on PHP 8.2.
===config===
php_version=8.2
suppress=UnusedVariable
===file===
<?php

$document = new DOMDocument();
$schema = '<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" />';

$defaultFlags = $document->schemaValidateSource($schema);
$namedFlags = $document->schemaValidateSource(source: $schema, flags: 0);
$missingSource = $document->schemaValidateSource();
===expect===
TooFewArguments@8:17-8:50: Too few arguments for schemaValidateSource(): expected 1, got 0

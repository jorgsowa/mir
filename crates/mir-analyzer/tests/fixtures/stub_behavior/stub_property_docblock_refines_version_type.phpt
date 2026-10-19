===description===
A stub property's @var docblock refines its #[LanguageLevelTypeAware] type (generic params,
class-string) but never drops the attribute's nullability or widens past it.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function dom(DOMNode $node, DOMDocument $document, DOMDocumentType $doctype, DOMAttr $attr): void {
    $children = $node->childNodes;
    /** @mir-check $children is DOMNodeList<DOMNode> */
    $entities = $doctype->entities;
    /** @mir-check $entities is DOMNamedNodeMap<DOMEntity> */
    $notations = $doctype->notations;
    /** @mir-check $notations is DOMNamedNodeMap<DOMNotation> */
    $root = $document->documentElement;
    /** @mir-check $root is DOMElement|null */
    $owner = $attr->ownerElement;
    /** @mir-check $owner is DOMElement|null */
    $prefix = $node->prefix;
    /** @mir-check $prefix is string */
    $typeInfo = $attr->schemaTypeInfo;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $typeInfo is assigned a mixed type
}

/** @param ReflectionClass<Exception> $class */
function reflection(ReflectionClass $class, ReflectionMethod $method, mysqli $db): void {
    $name = $class->name;
    /** @mir-check $name is class-string<Exception> */
    $declaring = $method->class;
    /** @mir-check $declaring is class-string */
    $rows = $db->affected_rows;
    /** @mir-check $rows is int|string */
}

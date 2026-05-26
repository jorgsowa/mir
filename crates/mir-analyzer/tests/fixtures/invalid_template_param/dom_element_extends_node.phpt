===description===
DOMElement (subclass of DOMNode) satisfies template bound of DOMNode - regression for issue E
===file===
<?php
$el = new DOMElement('div');
$parent = new DOMNode();

// DOMElement extends DOMNode, so it should satisfy TNode bound
$parent->appendChild($el);
$parent->insertBefore($el, null);
$parent->replaceChild($el, $parent->firstChild);

// Same with DOMComment
$comment = new DOMComment('test');
$parent->appendChild($comment);

// And DOMText
$text = new DOMText('test');
$parent->appendChild($text);

// Test with DOMDocumentFragment
$fragment = new DOMDocumentFragment();
$fragment->appendChild($el);
===expect===

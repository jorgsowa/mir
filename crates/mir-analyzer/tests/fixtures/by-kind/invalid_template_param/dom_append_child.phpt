===description===
DOM appendChild with DOMElement (subclass of DOMNode) should satisfy template bound
===file===
<?php
$el = new DOMElement('x');
$parent = new DOMNode();
$parent->appendChild($el);
===expect===

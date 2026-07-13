===description===
Sibling of anonymous_class_extends_final_class: extending a non-final base
must stay silent.
===file===
<?php

class Base {}

new class extends Base {};
===expect===

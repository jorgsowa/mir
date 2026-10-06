===description===
DuplicateFunction fires for a namespaced function declared twice in the same file.
===file===
<?php
namespace App;

function greet(): string { return 'hello'; }
function greet(): string { return 'hi'; }
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DuplicateFunction: Function App\greet() has already been defined

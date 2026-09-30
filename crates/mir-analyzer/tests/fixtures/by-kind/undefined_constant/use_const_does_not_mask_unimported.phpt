===description===
An unimported name stays undefined; an import doesn't resolve a different name.
===file===
<?php
namespace App;

use const ast\AST_CLASS as KIND;

function f(): int {
    return KIND + AST_CLASS;
//  ^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
//                ^^^^^^^^^ UndefinedConstant: Constant AST_CLASS is not defined
}
===expect===

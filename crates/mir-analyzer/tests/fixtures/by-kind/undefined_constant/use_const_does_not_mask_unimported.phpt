===description===
An unimported name stays undefined; an import doesn't resolve a different name.
===file===
<?php
namespace App;

use const ast\AST_CLASS as KIND;

function f(): int {
    return KIND + AST_CLASS;
}
===expect===
MixedReturnStatement@7:4-7:28: Cannot return a mixed type from function with declared return type 'int'
UndefinedConstant@7:18-7:27: Constant AST_CLASS is not defined

===description===
Detect unused variable inside loop after assignment with addition
===file===
<?php
function foo() : void {
    foreach ([1, 2, 3] as $i) {
        $i = $i + 1;
//      ^^ UnusedForeachValue: Foreach value $i is never read
    }
}
===expect===

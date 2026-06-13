===description===
Regression (laravel/framework): declared `@return HtmlString|void` with a bare
`return;` is valid. The bare-return guard now checks `contains(TVoid)` (not just
the single-atomic `is_void()`), so void-in-a-union no longer emits
InvalidReturnType.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
class HtmlString {}
/**
 * @return HtmlString|void
 */
function render(bool $cond) {
    if (! $cond) {
        return;
    }
    return new HtmlString();
}
===expect===

===description===
Laravel FP (laravel/framework): declared `@return HtmlString|void` with a bare
`return;` is valid, but mir's is_void() only matches a single TVoid atomic (misses
void-in-a-union), so it emits InvalidReturnType. Ignored pending fix — see ROADMAP
§1.4.
===ignore===
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

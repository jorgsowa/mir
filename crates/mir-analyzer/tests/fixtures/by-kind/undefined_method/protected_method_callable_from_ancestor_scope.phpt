===description===
A protected method is callable from an ancestor of its declaring class, as PHP
allows, including from a closure scoped to the ancestor that narrows `$this` to
a subclass overriding the method.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class CliDumper {
    protected string $line = '';
    protected function style(string $s): string { return $s; }
    public function describe(HtmlDumper $html): string {
        return $html->style('x') . $html->extra();
    }
}
class HtmlDumper extends CliDumper {
    protected function style(string $s): string { return "<b>$s</b>"; }
    protected function extra(): string { return ''; }
}
class Unrelated {
    protected function style(string $s): string { return $s; }
}
class Collector {
    public function dump(CliDumper $dumper, Unrelated $other) {
        $f = function (string $x) {
            if ($this instanceof HtmlDumper) {
                $this->line = $this->style($x);
            }
            $this->line .= $this->style($x);
        };
        $f = $f->bindTo($dumper, $dumper);
        $other->style('x');
//      ^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Unrelated::style() does not exist
        return $f;
    }
}

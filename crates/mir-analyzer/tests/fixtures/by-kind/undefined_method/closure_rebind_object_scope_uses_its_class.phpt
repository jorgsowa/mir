===description===
A closure rebound with an object variable as `$newScope` is scoped to that
variable's class, so its protected and private members stay accessible after
`$this` is narrowed to a subclass.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Dumper {
    protected string $line = '';
    protected function style(string $s): string { return $s; }
}
class HtmlDumper extends Dumper {}
class Host {
    public function assignThenBindTo(Dumper $d) {
        $f = function (string $x) {
            $this->line = $this->style($x);
            if ($this instanceof HtmlDumper) {
                $this->line .= $this->style($x);
            }
        };
        return $f->bindTo($d, $d);
    }
    public function inlineBindTo(Dumper $d) {
        return (function () {
            if ($this instanceof HtmlDumper) {
                return $this->style($this->line);
            }
            return '';
        })->bindTo($d, $d);
    }
    public function staticBind(Dumper $d) {
        return Closure::bind(function () {
            if ($this instanceof HtmlDumper) {
                return $this->style($this->line);
            }
            return '';
        }, $d, $d);
    }
    public function assignThenStaticBind(Dumper $d) {
        $f = function () {
            if ($this instanceof HtmlDumper) {
                return $this->style($this->line);
            }
            return '';
        };
        return Closure::bind($f, $d, $d);
    }
    public function missingMethodStillReported(Dumper $d) {
        return (function () { return $this->missing(); })->bindTo($d, $d);
//                                   ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Dumper::missing() does not exist
    }
    public function nullableScopeLeavesThisUnknown(?Dumper $d) {
        return (function () { return $this->missing(); })->bindTo($d, $d);
    }
}

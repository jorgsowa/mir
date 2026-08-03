===description===
M18: Throwable/Exception/Error::getTrace() were typed bare `array`, so
each frame was `mixed` and indexing into it (e.g. ['function']) flagged
MixedArrayAccess. Fixed by giving getTrace() the shaped
list<array{function: string, line?: int, file?: string, class?:
class-string, type?: string, args?: list<mixed>, object?: object}>
return type Psalm itself ships.
===file===
<?php
function firstFrameFunction(Throwable $e): string {
    $trace = $e->getTrace();

    return $trace[0]['function'];
}
===expect===

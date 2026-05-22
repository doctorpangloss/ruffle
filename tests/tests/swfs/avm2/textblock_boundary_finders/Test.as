package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans"; fd.fontLookup = FontLookup.DEVICE;
        var ef:ElementFormat = new ElementFormat(fd, 20);
        var tb:TextBlock = new TextBlock(new TextElement("Hello world foo", ef));
        // Plain single-space text.
        trace("nextAtom(0): " + tb.findNextAtomBoundary(0));
        trace("nextAtom(5): " + tb.findNextAtomBoundary(5));
        trace("nextAtom(14): " + tb.findNextAtomBoundary(14));
        trace("prevAtom(1): " + tb.findPreviousAtomBoundary(1));
        trace("prevAtom(14): " + tb.findPreviousAtomBoundary(14));
        trace("nextWord(0): " + tb.findNextWordBoundary(0));
        trace("nextWord(5): " + tb.findNextWordBoundary(5));
        trace("nextWord(7): " + tb.findNextWordBoundary(7));
        trace("prevWord(7): " + tb.findPreviousWordBoundary(7));
        trace("prevWord(14): " + tb.findPreviousWordBoundary(14));
        // Multi-space text "aa  bb  cc" — exercises the asymmetric whitespace rule.
        var tb2:TextBlock = new TextBlock(new TextElement("aa  bb  cc", ef));
        trace("multi nextWord(2): " + tb2.findNextWordBoundary(2));
        trace("multi nextWord(3): " + tb2.findNextWordBoundary(3));
        trace("multi nextWord(4): " + tb2.findNextWordBoundary(4));
        trace("multi prevWord(2): " + tb2.findPreviousWordBoundary(2));
        trace("multi prevWord(3): " + tb2.findPreviousWordBoundary(3));
        trace("multi prevWord(4): " + tb2.findPreviousWordBoundary(4));
        trace("multi prevWord(6): " + tb2.findPreviousWordBoundary(6));
        trace("multi prevWord(7): " + tb2.findPreviousWordBoundary(7));
        // Boundary throws.
        check("nextAtom(-1)", function():void { tb.findNextAtomBoundary(-1); });
        check("nextAtom(15)", function():void { tb.findNextAtomBoundary(15); });
        check("prevAtom(0)",  function():void { tb.findPreviousAtomBoundary(0); });
        check("prevAtom(15)", function():void { tb.findPreviousAtomBoundary(15); });
        check("nextWord(15)", function():void { tb.findNextWordBoundary(15); });
        check("prevWord(0)",  function():void { tb.findPreviousWordBoundary(0); });
        check("prevWord(15)", function():void { tb.findPreviousWordBoundary(15); });
    }
    private function check(label:String, fn:Function):void {
        try { fn(); trace(label + ": no throw"); }
        catch (e:RangeError)    { trace(label + ": RangeError #" + e.errorID); }
        catch (e:ArgumentError) { trace(label + ": ArgumentError #" + e.errorID); }
        catch (e:Error)         { trace(label + ": " + e); }
    }
}
}

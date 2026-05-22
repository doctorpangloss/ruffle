package {
import flash.display.Sprite;
import flash.text.engine.*;

// Liberation Sans is the system font Flash Player picks up on Linux
// when an SWF asks for it by name. Its OS/2 typographic ascender is
// 1491 and descender is -431. Its hhea ascender is 1854 and descender
// is -434. At UPM 2048 and size 100, that comes out to 72.80 and
// -21.04 if Flash reads the OS/2 typo fields, or 90.53 and -21.19 if
// Flash reads the hhea fields.
//
// The trace rounds to a quarter pixel so Flash's sub-twip precision
// and Ruffle's twip-quantised value agree exactly. 72.75 versus 90.5
// is still wide enough that the trace says unambiguously which set
// of fields Flash used.
[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        fd.fontLookup = FontLookup.DEVICE;
        var ef:ElementFormat = new ElementFormat(fd, 100);
        var tb:TextBlock = new TextBlock(new TextElement("Ag", ef));
        var line:TextLine = tb.createTextLine(null, 100000);
        trace("ascent: " + q(line.ascent));
        trace("descent: " + q(line.descent));
        line.x = 0;
        line.y = line.ascent;
        addChild(line);
    }

    private function q(n:Number):Number {
        return Math.round(n * 4) / 4;
    }
}
}

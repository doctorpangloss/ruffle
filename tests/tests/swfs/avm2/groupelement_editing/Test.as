package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="200", height="100")]
public class Test extends Sprite {
    public function Test() {
        trace("step 1: init");
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans"; fd.fontLookup = FontLookup.DEVICE;
        var ef:ElementFormat = new ElementFormat(fd, 20);
        trace("step 2: format created");
        var children:Vector.<ContentElement> = new Vector.<ContentElement>();
        children.push(new TextElement("aaa", ef));
        children.push(new TextElement("bbb", ef));
        children.push(new TextElement("ccc", ef));
        children.push(new TextElement("ddd", ef));
        trace("step 3: children built, length=" + children.length);
        var g:GroupElement = new GroupElement(children, ef);
        trace("step 4: group created, elementCount=" + g.elementCount);
        trace("text: " + g.text);
        try { trace("getElAt(0): " + (g.getElementAtCharIndex(0) as TextElement).text); } catch (e:Error) { trace("getElAt(0) THROW: " + e); }
        try { trace("getElAt(5): " + (g.getElementAtCharIndex(5) as TextElement).text); } catch (e:Error) { trace("getElAt(5) THROW: " + e); }
        try { trace("getElAt(8): " + (g.getElementAtCharIndex(8) as TextElement).text); } catch (e:Error) { trace("getElAt(8) THROW: " + e); }
        try {
            g.groupElements(1, 3);
            trace("groupElements done, elementCount=" + g.elementCount);
            var inner:ContentElement = g.getElementAt(1);
            trace("inner is GroupElement: " + (inner is GroupElement));
            if (inner is GroupElement) { trace("inner text: " + (inner as GroupElement).text); }
        } catch (e:Error) { trace("groupElements THROW: " + e); }
        try {
            g.ungroupElements(1);
            trace("ungroupElements done, elementCount=" + g.elementCount);
        } catch (e:Error) { trace("ungroupElements THROW: " + e); }
        try {
            g.mergeTextElements(0, 2);
            trace("mergeTextElements done, elementCount=" + g.elementCount);
            trace("merged text: " + (g.getElementAt(0) as TextElement).text);
        } catch (e:Error) { trace("mergeTextElements THROW: " + e); }
    }
}
}

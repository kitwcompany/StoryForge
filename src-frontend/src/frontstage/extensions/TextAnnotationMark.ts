import { Mark, mergeAttributes } from '@tiptap/core';

export interface TextAnnotationMarkOptions {
  HTMLAttributes: Record<string, any>;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    textAnnotation: {
      setTextAnnotation: (attributes: {
        type: string;
        annotationId: string;
        severity?: string;
      }) => ReturnType;
      unsetTextAnnotation: () => ReturnType;
    };
  }
}

export const TextAnnotationMark = Mark.create<TextAnnotationMarkOptions>({
  name: 'textAnnotation',

  addOptions() {
    return {
      HTMLAttributes: {},
    };
  },

  addAttributes() {
    return {
      type: {
        default: 'note',
        parseHTML: element => element.getAttribute('data-annotation-type'),
        renderHTML: attributes => ({
          'data-annotation-type': attributes.type,
        }),
      },
      annotationId: {
        default: null,
        parseHTML: element => element.getAttribute('data-annotation-id'),
        renderHTML: attributes => ({
          'data-annotation-id': attributes.annotationId,
        }),
      },
      severity: {
        default: null,
        parseHTML: element => element.getAttribute('data-severity'),
        renderHTML: attributes => {
          if (!attributes.severity) return {};
          return { 'data-severity': attributes.severity };
        },
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-annotation-id]',
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    const type = HTMLAttributes['data-annotation-type'] || 'note';
    const severity = HTMLAttributes['data-severity'];

    // 静态类型颜色
    const colorMap: Record<string, string> = {
      note: 'rgba(59, 130, 246, 0.25)',
      todo: 'rgba(249, 115, 22, 0.25)',
      warning: 'rgba(239, 68, 68, 0.25)',
      idea: 'rgba(168, 85, 247, 0.25)',
    };
    const borderColorMap: Record<string, string> = {
      note: 'rgba(59, 130, 246, 0.6)',
      todo: 'rgba(249, 115, 22, 0.6)',
      warning: 'rgba(239, 68, 68, 0.6)',
      idea: 'rgba(168, 85, 247, 0.6)',
    };

    // ai_audit 类型按 severity 动态着色（Phase 0 实证：memory 维度优先醒目）
    let bgColor = colorMap[type] || colorMap.note;
    let borderColor = borderColorMap[type] || borderColorMap.note;
    if (type === 'ai_audit' && severity) {
      const severityColors: Record<string, { bg: string; border: string }> = {
        high: { bg: 'rgba(220, 38, 38, 0.18)', border: 'rgba(220, 38, 38, 0.7)' },
        medium: { bg: 'rgba(217, 119, 6, 0.18)', border: 'rgba(217, 119, 6, 0.6)' },
        low: { bg: 'rgba(59, 130, 246, 0.15)', border: 'rgba(59, 130, 246, 0.5)' },
      };
      const sc = severityColors[severity] || severityColors.medium;
      bgColor = sc.bg;
      borderColor = sc.border;
    }

    return [
      'span',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        style: `background-color: ${bgColor}; border-bottom: 2px solid ${borderColor}; cursor: pointer; border-radius: 2px;`,
      }),
      0,
    ];
  },

  addCommands() {
    return {
      setTextAnnotation:
        attributes =>
        ({ commands }) => {
          return commands.setMark(this.name, attributes);
        },
      unsetTextAnnotation:
        () =>
        ({ commands }) => {
          return commands.unsetMark(this.name);
        },
    };
  },
});

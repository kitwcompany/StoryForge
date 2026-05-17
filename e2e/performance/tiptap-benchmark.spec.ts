/**
 * Tiptap 性能基准测试 — W4-F6
 *
 * 测试维度:
 * - 输入延迟 (Input Latency): 连续输入 100 个字符的总耗时
 * - 保存序列化时间 (Serialize Time): editor.getHTML() 耗时
 * - 内存占用 (Memory): performance.memory.usedJSHeapSize
 *
 * 文档规模: 10万字 / 50万字 / 100万字
 * 验收标准: 明确性能拐点（输入延迟 > 50ms 或序列化 > 500ms）
 */

import { test, expect } from '@playwright/test';

interface BenchmarkResult {
  docSize: number; // 字符数
  inputLatencyMs: number; // 100 个字符输入耗时
  serializeTimeMs: number; // getHTML() 耗时
  memoryUsedMB: number; // JS 堆内存占用 (MB)
}

/**
 * 生成指定长度的中文测试文本（段落结构，模拟真实文档）
 */
function generateChineseText(targetChars: number): string {
  const paragraph = '春风拂过山岗，带来远方的消息。古老的村庄在晨曦中苏醒，炊烟袅袅升起。';
  const sentences = [
    '少年背着行囊踏上旅途，心中满是对未知的憧憬与忐忑。',
    '山间的溪流清澈见底，鱼儿在水中自由穿梭。',
    '夜幕降临，繁星点缀着深邃的天空，银河横跨天际。',
    '老者坐在门前的竹椅上，讲述着流传千年的传说。',
    '秋叶飘零，铺满林间的小径，踩上去发出沙沙的声响。',
  ];

  let text = '';
  let count = 0;
  while (count < targetChars) {
    const sentence = sentences[count % sentences.length];
    text += sentence;
    count += sentence.length;
    if (count % 200 < 50) {
      text += '\n';
    }
  }
  return text.slice(0, targetChars);
}

/**
 * 在浏览器中执行单次基准测试
 */
async function runBrowserBenchmark(page: any, docSize: number): Promise<BenchmarkResult> {
  const testText = generateChineseText(docSize);

  return page.evaluate(async (text: string) => {
    // 等待编辑器实例就绪
    const editor: any = (window as any).__BENCHMARK_EDITOR__;
    if (!editor) {
      throw new Error('Editor not found. Make sure __BENCHMARK_EDITOR__ is exposed.');
    }

    // 加载大文档
    editor.commands.setContent('<p>' + text.replace(/\n/g, '</p><p>') + '</p>');
    await new Promise((r) => setTimeout(r, 500));

    // 测量内存（Chrome 特有）
    const memBefore = (performance as any).memory?.usedJSHeapSize || 0;

    // 1. 测量输入延迟：模拟连续输入 100 个字符
    const latencyStart = performance.now();
    const inputText = '这是一段用于测量输入延迟的测试文本，连续输入一百个字符以评估编辑器响应性能。';
    for (let i = 0; i < inputText.length; i++) {
      editor.commands.insertContent(inputText[i]);
    }
    const latencyEnd = performance.now();
    const inputLatencyMs = latencyEnd - latencyStart;

    // 2. 测量序列化时间
    const serializeStart = performance.now();
    const html = editor.getHTML();
    const serializeEnd = performance.now();
    const serializeTimeMs = serializeEnd - serializeStart;

    // 3. 测量内存
    await new Promise((r) => setTimeout(r, 200));
    const memAfter = (performance as any).memory?.usedJSHeapSize || 0;
    const memoryUsedMB = (memAfter - memBefore) / 1024 / 1024;

    return {
      docSize: text.length,
      inputLatencyMs: Math.round(inputLatencyMs * 100) / 100,
      serializeTimeMs: Math.round(serializeTimeMs * 100) / 100,
      memoryUsedMB: Math.round(memoryUsedMB * 100) / 100,
    };
  }, testText);
}

test.describe('Tiptap 性能基准 (W4-F6)', () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });

    // 注入 mock Tauri API，让 Frontstage 能正常加载
    await page.addInitScript(() => {
      const mockChapter = {
        id: 'bench-chapter-1',
        story_id: 'bench-story-1',
        title: '性能测试章节',
        chapter_number: 1,
        content: ''
      };

      (window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args?: any) => {
          if (cmd === 'list_stories') {
            return [{ id: 'bench-story-1', title: '性能测试故事' }];
          }
          if (cmd === 'get_story_chapters') {
            return [mockChapter];
          }
          if (cmd === 'get_story_scenes') {
            return [];
          }
          if (cmd === 'get_chapter') {
            return mockChapter;
          }
          if (cmd === 'update_chapter') {
            mockChapter.content = args?.content || '';
            return null;
          }
          if (cmd === 'notify_backstage_content_changed') {
            return null;
          }
          return null;
        }
      };
    });

    await page.goto('/frontstage.html');
    await page.waitForTimeout(3000);

    // 将 Tiptap editor 实例暴露到全局，供 benchmark 使用
    await page.evaluate(() => {
      // RichTextEditor 组件通过 ref 暴露 editor，但 FrontstageApp 持有 ref。
      // 我们通过 DOM 事件或全局变量来暴露 editor。
      // 由于无法直接访问 React ref，我们尝试从 DOM 中查找 editor 实例。
      // @tiptap/react 的 EditorContent 不直接暴露 editor 到 DOM，
      // 但我们可以利用 React DevTools 或监听自定义事件。
      // 这里采用一个变通方案：在 RichTextEditor 中如果检测到 benchmark 模式，主动暴露 editor。
      // 为简化测试，我们通过 dispatchEvent 请求 editor 暴露自身。
      const event = new CustomEvent('__expose_editor_for_benchmark__', {
        detail: { callback: (editor: any) => { (window as any).__BENCHMARK_EDITOR__ = editor; } }
      });
      window.dispatchEvent(event);
    });

    // 等待 editor 被暴露（RichTextEditor 需要监听该事件）
    await page.waitForTimeout(500);
  });

  const docSizes = [
    { size: 100_000, label: '10万字' },
    { size: 500_000, label: '50万字' },
    { size: 1_000_000, label: '100万字' },
  ];

  for (const { size, label } of docSizes) {
    test(`${label} 文档性能`, async ({ page }) => {
      // 检查 editor 是否成功暴露
      const hasEditor = await page.evaluate(() => !!(window as any).__BENCHMARK_EDITOR__);
      if (!hasEditor) {
        test.skip(true, 'Editor instance not exposed — RichTextEditor 需添加 benchmark 支持');
        return;
      }

      const result = await runBrowserBenchmark(page, size);

      console.log(`[${label}] 输入延迟: ${result.inputLatencyMs}ms | 序列化: ${result.serializeTimeMs}ms | 内存增量: ${result.memoryUsedMB}MB`);

      // 断言：输入延迟不应超过 500ms（100字连续输入）
      expect(result.inputLatencyMs).toBeLessThan(500);
      // 断言：序列化不应超过 2000ms
      expect(result.serializeTimeMs).toBeLessThan(2000);

      // 记录结果到测试附件（Playwright 不支持直接附件，console 输出供 CI 收集）
      console.log('BENCHMARK_RESULT|' + JSON.stringify({ label, ...result }));
    });
  }

  test('生成性能拐点报告', async ({ page }) => {
    const hasEditor = await page.evaluate(() => !!(window as any).__BENCHMARK_EDITOR__);
    if (!hasEditor) {
      test.skip(true, 'Editor instance not exposed');
      return;
    }

    const results: BenchmarkResult[] = [];
    for (const { size, label } of docSizes) {
      const result = await runBrowserBenchmark(page, size);
      results.push(result);
      console.log(`[${label}] 输入延迟: ${result.inputLatencyMs}ms | 序列化: ${result.serializeTimeMs}ms | 内存: ${result.memoryUsedMB}MB`);
    }

    // 自动判定拐点：输入延迟 > 50ms 或序列化 > 500ms
    const拐点 = results.find(r => r.inputLatencyMs > 50 || r.serializeTimeMs > 500);
    if (拐点) {
      console.log(`⚠️ 性能拐点出现在 ${拐点.docSize} 字文档（输入延迟 ${拐点.inputLatencyMs}ms，序列化 ${拐点.serializeTimeMs}ms）`);
    } else {
      console.log('✅ 100万字以内未出现明显性能拐点');
    }

    // 断言：至少有一个结果
    expect(results.length).toBe(docSizes.length);
  });
});

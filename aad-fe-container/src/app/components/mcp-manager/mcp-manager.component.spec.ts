import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { McpManagerComponent } from './mcp-manager.component';

describe('McpManagerComponent', () => {
  let component: McpManagerComponent;
  let fixture: ComponentFixture<McpManagerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [McpManagerComponent],
      providers: [provideHttpClient(), provideAnimationsAsync()]
    })
    .compileComponents();

    fixture = TestBed.createComponent(McpManagerComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

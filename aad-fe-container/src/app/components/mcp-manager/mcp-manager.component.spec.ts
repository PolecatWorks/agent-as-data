import { ComponentFixture, TestBed } from '@angular/core/testing';

import { McpManagerComponent } from './mcp-manager.component';

describe('McpManagerComponent', () => {
  let component: McpManagerComponent;
  let fixture: ComponentFixture<McpManagerComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [McpManagerComponent]
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

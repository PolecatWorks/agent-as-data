import { HttpClientTestingModule } from '@angular/common/http/testing';
import { RouterTestingModule } from '@angular/router/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ToolEmptyComponent } from './tool-empty.component';

describe('ToolEmptyComponent', () => {
  let component: ToolEmptyComponent;
  let fixture: ComponentFixture<ToolEmptyComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HttpClientTestingModule, RouterTestingModule, ToolEmptyComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ToolEmptyComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
